import { createRouter, createWebHistory } from 'vue-router'

import AppShell from '../components/AppShell.vue'
import AboutView from '../views/AboutView.vue'
import AcknowledgementsView from '../views/AcknowledgementsView.vue'
import CasesView from '../views/CasesView.vue'
import ForgotPasswordView from '../views/ForgotPasswordView.vue'
import LoginView from '../views/LoginView.vue'
import PracticeStubView from '../views/PracticeStubView.vue'
import PrivacyView from '../views/PrivacyView.vue'
import ProgressStubView from '../views/ProgressStubView.vue'
import RegisterView from '../views/RegisterView.vue'
import ResetPasswordView from '../views/ResetPasswordView.vue'
import SettingsView from '../views/SettingsView.vue'
import TermsView from '../views/TermsView.vue'
import VerifyEmailView from '../views/VerifyEmailView.vue'
import { useAuthStore } from '../stores/auth'

declare module 'vue-router' {
  interface RouteMeta {
    /// Route is only reachable when authenticated; unauthed visitors are
    /// redirected to /login?next=<original>.
    requiresAuth?: boolean
    /// Route is only reachable when unauthenticated; authed visitors are
    /// bounced back to / (or `?next=`).
    guestOnly?: boolean
  }
}

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    // Tabbed app shell — Practice (default) / Cases / Progress live as
    // children so the bottom tab bar is shared. Auth-required at the
    // shell level; checked via `to.matched.some()` in the guard.
    {
      path: '/',
      component: AppShell,
      meta: { requiresAuth: true },
      children: [
        { path: '', name: 'practice', component: PracticeStubView },
        { path: 'cases', name: 'cases', component: CasesView },
        { path: 'progress', name: 'progress', component: ProgressStubView },
      ],
    },
    // Settings is full-bleed — no tab bar, has its own back button.
    { path: '/settings', name: 'settings', component: SettingsView, meta: { requiresAuth: true } },

    // Auth views.
    { path: '/login', name: 'login', component: LoginView, meta: { guestOnly: true } },
    {
      path: '/register',
      name: 'register',
      component: RegisterView,
      meta: { guestOnly: true },
    },
    {
      path: '/forgot-password',
      name: 'forgot-password',
      component: ForgotPasswordView,
      meta: { guestOnly: true },
    },
    {
      path: '/reset-password',
      name: 'reset-password',
      component: ResetPasswordView,
      meta: { guestOnly: true },
    },
    // Verify-email is reachable from both states: post-registration (no session yet)
    // and during an email change (session present). No meta — the view handles both.
    { path: '/verify-email', name: 'verify-email', component: VerifyEmailView },

    // Public static pages — placeholder content until launch.
    { path: '/about', name: 'about', component: AboutView },
    { path: '/terms', name: 'terms', component: TermsView },
    { path: '/privacy', name: 'privacy', component: PrivacyView },
    {
      path: '/acknowledgements',
      name: 'acknowledgements',
      component: AcknowledgementsView,
    },
  ],
})

router.beforeEach(async (to) => {
  const auth = useAuthStore()
  // Wait for the initial /auth/me round-trip before applying any guard logic.
  // bootstrap() is idempotent — same promise is reused after the first call.
  await auth.bootstrap()

  // Walk matched records so parent route's requiresAuth applies to children.
  const requiresAuth = to.matched.some((r) => r.meta.requiresAuth)
  if (requiresAuth && auth.status !== 'authed') {
    return { path: '/login', query: { next: to.fullPath } }
  }

  if (to.meta.guestOnly && auth.status === 'authed') {
    const nextRaw = to.query.next
    const next = typeof nextRaw === 'string' && nextRaw.startsWith('/') ? nextRaw : '/'
    return next
  }
})

export default router
