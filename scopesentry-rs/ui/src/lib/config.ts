// Prefer same-origin during dev with Vite proxy; fallback to explicit env
export const API_BASE = (import.meta.env.VITE_API_BASE as string) || ''
