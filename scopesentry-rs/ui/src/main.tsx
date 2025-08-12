import React from 'react'
import ReactDOM from 'react-dom/client'
import { createBrowserRouter, RouterProvider } from 'react-router-dom'
import './index.css'
import { AppLayout } from './pages/_layout'
import { Dashboard } from './pages/dashboard'
import { NodesPage } from './pages/nodes'
import { CreateTaskPage } from './pages/task-create'

const router = createBrowserRouter([
  {
    path: '/',
    element: <AppLayout />,
    children: [
      { index: true, element: <Dashboard /> },
      { path: 'nodes', element: <NodesPage /> },
      { path: 'tasks/create', element: <CreateTaskPage /> },
    ],
  },
])

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <RouterProvider router={router} />
  </React.StrictMode>,
)
