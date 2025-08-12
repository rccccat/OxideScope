import { useEffect, useMemo, useState } from 'react'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Textarea } from '@/components/ui/textarea'
import { Select } from '@/components/ui/select'
import { createTask, getNodesOnline } from '@/lib/api'

export function CreateTaskPage() {
  const [name, setName] = useState('')
  const [target, setTarget] = useState('')
  const [ignore, setIgnore] = useState('')
  const [allNode, setAllNode] = useState(true)
  const [node, setNode] = useState<string[]>([])
  const [template, setTemplate] = useState('')
  const [duplicates, setDuplicates] = useState(false)
  const [scheduledTasks, setScheduledTasks] = useState(false)

  const [nodeOptions, setNodeOptions] = useState<string[]>([])
  const [submitting, setSubmitting] = useState(false)
  const [message, setMessage] = useState<string | null>(null)

  useEffect(() => {
    getNodesOnline().then((d) => setNodeOptions(d.list)).catch((e) => setMessage(String(e)))
  }, [])

  const canSubmit = useMemo(() => name.trim() && target.trim(), [name, target])

  async function onSubmit(e: React.FormEvent) {
    e.preventDefault()
    setSubmitting(true)
    setMessage(null)
    try {
      await createTask({ name, target, ignore, node, allNode, scheduledTasks, template, duplicates })
      setMessage('创建成功')
    } catch (e) {
      setMessage(String(e))
    } finally {
      setSubmitting(false)
    }
  }

  function toggleNode(n: string) {
    setNode((prev) => (prev.includes(n) ? prev.filter((x) => x !== n) : [...prev, n]))
  }

  return (
    <form className="max-w-3xl grid gap-6" onSubmit={onSubmit}>
      <div className="grid gap-2">
        <Label>任务名称</Label>
        <Input value={name} onChange={(e) => setName(e.target.value)} placeholder="例如：Acme 日常扫描" />
      </div>

      <div className="grid gap-2">
        <Label>目标（支持多行域名/IP/URL）</Label>
        <Textarea value={target} onChange={(e) => setTarget(e.target.value)} rows={6} placeholder="example.com\n1.2.3.4\nhttps://app.example.com" />
      </div>

      <div className="grid gap-2">
        <Label>忽略（可选）</Label>
        <Textarea value={ignore} onChange={(e) => setIgnore(e.target.value)} rows={3} placeholder="test.example.com" />
      </div>

      <div className="grid gap-2">
        <Label>模板 ID（可选）</Label>
        <Input value={template} onChange={(e) => setTemplate(e.target.value)} placeholder="Mongo ObjectId" />
      </div>

      <div className="grid gap-2">
        <Label>节点选择</Label>
        <div className="flex items-center gap-3">
          <label className="flex items-center gap-2 text-sm">
            <input type="checkbox" checked={allNode} onChange={(e) => setAllNode(e.target.checked)} />
            全部在线节点
          </label>
          <label className="flex items-center gap-2 text-sm">
            <input type="checkbox" checked={duplicates} onChange={(e) => setDuplicates(e.target.checked)} />
            去重
          </label>
          <label className="flex items-center gap-2 text-sm">
            <input type="checkbox" checked={scheduledTasks} onChange={(e) => setScheduledTasks(e.target.checked)} />
            定时任务
          </label>
        </div>
        {!allNode && (
          <div className="grid grid-cols-2 gap-2 border rounded-md p-3">
            {nodeOptions.map((n) => (
              <label key={n} className="flex items-center gap-2 text-sm">
                <input type="checkbox" checked={node.includes(n)} onChange={() => toggleNode(n)} />
                {n}
              </label>
            ))}
          </div>
        )}
      </div>

      <div className="flex items-center gap-3">
        <Button type="submit" disabled={!canSubmit || submitting}>{submitting ? '提交中...' : '创建任务'}</Button>
        {message && <span className="text-sm text-muted-foreground">{message}</span>}
      </div>
    </form>
  )
}
