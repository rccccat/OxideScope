import { Outlet, Link, NavLink, useLocation } from 'react-router-dom'
import { Button } from '@/components/ui/button'
import { cn } from '@/lib/utils'

export function AppLayout() {
  const { pathname } = useLocation()
  return (
    <div className="min-h-screen grid grid-rows-[auto_1fr]">
      <header className="border-b">
        <div className="container flex h-14 items-center justify-between">
          <div className="flex items-center gap-6">
            <Link to="/" className="font-semibold">ScopeSentry RS</Link>
            <nav className="flex items-center gap-4 text-sm">
              <NavItem to="/" active={pathname === '/'}>仪表盘</NavItem>
              <NavItem to="/nodes">节点</NavItem>
              <NavItem to="/tasks/create">创建任务</NavItem>
            </nav>
          </div>
          <div className="flex items-center gap-2">
            <Button variant="outline" size="sm" onClick={() => document.documentElement.classList.toggle('dark')}>主题</Button>
          </div>
        </div>
      </header>
      <main className="container py-6">
        <Outlet />
      </main>
    </div>
  )
}

function NavItem({ to, children, active }: { to: string; children: React.ReactNode; active?: boolean }) {
  return (
    <NavLink to={to} className={cn('text-muted-foreground hover:text-foreground', active && 'text-foreground')}>
      {children}
    </NavLink>
  )
}
