// Axios client for the Quiet Cube backend.
// - Base URL from VITE_API_BASE_URL (falls back to localhost:8080 for dev).
// - withCredentials so the httpOnly session cookie travels.
// - Error envelope `{ error, message, fields? }` is unwrapped into ApiError so
//   call sites can switch on the machine code without parsing axios's shape.

import axios, { AxiosError, type AxiosInstance } from 'axios'

export interface ApiErrorEnvelope {
  error: string
  message?: string
  fields?: Record<string, string>
  retry_after_seconds?: number
}

export class ApiError extends Error {
  code: string
  fields: Record<string, string>
  retryAfterSeconds?: number
  status: number

  constructor(envelope: ApiErrorEnvelope, status: number) {
    super(envelope.message ?? envelope.error)
    this.code = envelope.error
    this.fields = envelope.fields ?? {}
    this.retryAfterSeconds = envelope.retry_after_seconds
    this.status = status
  }
}

const baseURL = import.meta.env.VITE_API_BASE_URL ?? 'http://localhost:8080'

export const api: AxiosInstance = axios.create({
  baseURL: `${baseURL}/api/v1`,
  withCredentials: true,
  headers: { 'Content-Type': 'application/json' },
})

api.interceptors.response.use(
  (response) => response,
  (error: AxiosError<ApiErrorEnvelope>) => {
    if (error.response?.data?.error) {
      return Promise.reject(new ApiError(error.response.data, error.response.status))
    }
    // Network errors / no response — synthesize a generic envelope.
    return Promise.reject(
      new ApiError(
        { error: 'network', message: error.message || 'Network error.' },
        error.response?.status ?? 0,
      ),
    )
  },
)
