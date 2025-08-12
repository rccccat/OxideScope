import { useEffect, useState } from 'react'
import { getNodesOnline } from '@/lib/api'

export function NodesPage() {
  const [nodes, setNodes] = useState<string[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    let cancelled = false
    setLoading(true)
    getNodesOnline()
      .then((data) => { if (!cancelled) setNodes(data.list) })
      .catch((e) => { if (!cancelled) setError(String(e)) })
      .finally(() => { if (!cancelled) setLoading(false) })
    return () => { cancelled = true }
  }, [])

  if (loading) return <div>加载中...</div>
  if (error) return <div className="text-destructive">{error}</div>

  return (
    <div>
      <h2 className="text-lg font-semibold mb-4">在线节点</h2>
      <div className="grid gap-3">
        {nodes.length === 0 && <div className="text-muted-foreground">暂无在线节点</div>}
        {nodes.map((n) => (
          <div key={n} className="border rounded-md p-3 flex items-center justify-between">
            <div className="font-mono">{n}</div>
            <span className="text-xs rounded bg-green-100 text-green-700 px-2 py-1 dark:bg-green-900/40 dark:text-green-300">online</span>
          </div>
        ))}
      </div>
    </div>
  )
}
