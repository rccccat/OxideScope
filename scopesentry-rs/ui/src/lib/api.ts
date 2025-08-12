import { API_BASE } from './config'

export type ApiResponse<T> = { code: number; message?: string; data?: T }

async function http<T>(path: string, init?: RequestInit): Promise<T> {
  const res = await fetch(`${API_BASE}${path}`, {
    headers: { 'Content-Type': 'application/json' },
    ...init,
  })
  if (!res.ok) {
    throw new Error(`HTTP ${res.status}`)
  }
  const json = (await res.json()) as ApiResponse<T>
  if (json.code !== 200) {
    throw new Error(json.message || 'Request failed')
  }
  return json.data as T
}

export type NodesOnline = { list: string[] }
export function getNodesOnline() {
  return http<NodesOnline>('/api/node/data/online')
}

export type TaskAddRequest = {
  name: string
  target: string
  ignore?: string
  node: string[]
  allNode: boolean
  scheduledTasks: boolean
  template: string
  duplicates: boolean
}

export function createTask(body: TaskAddRequest) {
  return http<unknown>('/api/task/add', {
    method: 'POST',
    body: JSON.stringify(body),
  })
}
