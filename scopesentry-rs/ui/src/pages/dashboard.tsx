import { Card } from './partials/card'
import { Link } from 'react-router-dom'

export function Dashboard() {
  return (
    <div className="grid gap-6 md:grid-cols-2 lg:grid-cols-3">
      <Card title="在线节点" value={<Link className="underline" to="/nodes">查看</Link>} />
      <Card title="创建任务" value={<Link className="underline" to="/tasks/create">去创建</Link>} />
      <Card title="构建状态" value={<span className="text-muted-foreground">OK</span>} />
    </div>
  )
}
