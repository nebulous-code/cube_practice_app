// Cloudflare Turnstile helper.
// Loads Cloudflare's script lazily on first call, then exposes
// execute(action) → token. Internally renders an ephemeral hidden widget per
// call so the surface area matches the v3-style "request a token" pattern.
// Returns an empty token when VITE_TURNSTILE_SITE_KEY is unset — backend
// skips the check in the same condition (TURNSTILE_SECRET_KEY empty), so
// dev/curl roundtrips don't need a captcha.

const SITE_KEY = import.meta.env.VITE_TURNSTILE_SITE_KEY as string | undefined

interface TurnstileRenderOptions {
  sitekey: string
  action?: string
  appearance?: 'always' | 'execute' | 'interaction-only'
  callback: (token: string) => void
  'error-callback'?: () => void
  'timeout-callback'?: () => void
}

declare global {
  interface Window {
    turnstile?: {
      render: (
        container: HTMLElement | string,
        opts: TurnstileRenderOptions,
      ) => string | undefined
      remove: (widgetId: string) => void
    }
  }
}

let scriptPromise: Promise<void> | null = null

function loadScript(): Promise<void> {
  if (!SITE_KEY) return Promise.resolve()
  if (scriptPromise) return scriptPromise

  scriptPromise = new Promise<void>((resolve, reject) => {
    const existing = document.querySelector<HTMLScriptElement>(
      'script[data-turnstile-loader]',
    )
    if (existing) {
      existing.addEventListener('load', () => resolve())
      existing.addEventListener('error', () => reject(new Error('Turnstile load failed')))
      return
    }

    const script = document.createElement('script')
    script.src = 'https://challenges.cloudflare.com/turnstile/v0/api.js'
    script.async = true
    script.defer = true
    script.dataset.turnstileLoader = 'true'
    script.onload = () => resolve()
    script.onerror = () => reject(new Error('Turnstile load failed'))
    document.head.appendChild(script)
  })
  return scriptPromise
}

export async function executeTurnstile(action: string): Promise<string> {
  if (!SITE_KEY) return ''
  await loadScript()
  if (!window.turnstile) return ''

  return new Promise<string>((resolve) => {
    const container = document.createElement('div')
    container.style.position = 'fixed'
    container.style.left = '-10000px'
    container.style.top = '-10000px'
    document.body.appendChild(container)

    let widgetId: string | undefined
    const cleanup = () => {
      if (widgetId && window.turnstile) {
        try { window.turnstile.remove(widgetId) } catch { /* widget already gone */ }
      }
      container.remove()
    }

    // Script is already loaded (we awaited onload above), so render directly.
    // ready() is forbidden when the script tag has async/defer attributes.
    widgetId = window.turnstile!.render(container, {
      sitekey: SITE_KEY,
      action,
      appearance: 'interaction-only',
      callback: (token: string) => {
        cleanup()
        resolve(token)
      },
      'error-callback': () => {
        cleanup()
        resolve('')
      },
      'timeout-callback': () => {
        cleanup()
        resolve('')
      },
    })
    if (!widgetId) {
      cleanup()
      resolve('')
    }
  })
}
