import { createRouter, createWebHistory } from 'vue-router'

import HomeView from '../views/HomeView.vue'
import LoginView from '../views/LoginView.vue'
import RegisterView from '../views/RegisterView.vue'
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
    { path: '/', name: 'home', component: HomeView, meta: { requiresAuth: true } },
    { path: '/login', name: 'login', component: LoginView, meta: { guestOnly: true } },
    {
      path: '/register',
      name: 'register',
      component: RegisterView,
      meta: { guestOnly: true },
    },
    // Verify-email is reachable from both states: post-registration (no session yet)
    // and during an email change (session present). No meta — the view handles both.
    { path: '/verify-email', name: 'verify-email', component: VerifyEmailView },
  ],
})

router.beforeEach(async (to) => {
  const auth = useAuthStore()
  // Wait for the initial /auth/me round-trip before applying any guard logic.
  // bootstrap() is idempotent — same promise is reused after the first call.
  await auth.bootstrap()

  if (to.meta.requiresAuth && auth.status !== 'authed') {
    return { path: '/login', query: { next: to.fullPath } }
  }

  if (to.meta.guestOnly && auth.status === 'authed') {
    const nextRaw = to.query.next
    const next = typeof nextRaw === 'string' && nextRaw.startsWith('/') ? nextRaw : '/'
    return next
  }
})

export default router
