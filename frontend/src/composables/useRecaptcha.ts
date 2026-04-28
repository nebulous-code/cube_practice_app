// reCAPTCHA v3 helper.
// Loads Google's script lazily on first call, then exposes execute(action) → token.
// Returns an empty token when VITE_RECAPTCHA_SITE_KEY is unset — backend skips the
// check in the same condition (RECAPTCHA_SECRET_KEY empty), so dev/curl roundtrips
// don't need a captcha.

const SITE_KEY = import.meta.env.VITE_RECAPTCHA_SITE_KEY as string | undefined

declare global {
  interface Window {
    grecaptcha?: {
      ready: (cb: () => void) => void
      execute: (siteKey: string, opts: { action: string }) => Promise<string>
    }
  }
}

let scriptPromise: Promise<void> | null = null

function loadScript(): Promise<void> {
  if (!SITE_KEY) return Promise.resolve()
  if (scriptPromise) return scriptPromise

  scriptPromise = new Promise<void>((resolve, reject) => {
    const existing = document.querySelector<HTMLScriptElement>(
      'script[data-recaptcha-loader]',
    )
    if (existing) {
      existing.addEventListener('load', () => resolve())
      existing.addEventListener('error', () => reject(new Error('reCAPTCHA load failed')))
      return
    }

    const script = document.createElement('script')
    script.src = `https://www.google.com/recaptcha/api.js?render=${SITE_KEY}`
    script.async = true
    script.defer = true
    script.dataset.recaptchaLoader = 'true'
    script.onload = () => resolve()
    script.onerror = () => reject(new Error('reCAPTCHA load failed'))
    document.head.appendChild(script)
  })
  return scriptPromise
}

export async function executeRecaptcha(action: string): Promise<string> {
  if (!SITE_KEY) return ''
  await loadScript()
  if (!window.grecaptcha) return ''
  return new Promise<string>((resolve) => {
    window.grecaptcha!.ready(() => {
      window.grecaptcha!.execute(SITE_KEY, { action }).then(resolve).catch(() => resolve(''))
    })
  })
}
