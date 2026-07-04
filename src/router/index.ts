/** 路由配置——工作区子路由 + 设置页 */
import { createRouter, createWebHistory } from 'vue-router'
import type { RouteRecordRaw } from 'vue-router'

const routes: RouteRecordRaw[] = [
  {
    path: '/',
    redirect: '/workspace',
  },
  {
    path: '/workspace',
    component: () => import('@/layouts/MainLayout.vue'),
    children: [
      {
        path: '',
        name: 'HomePage',
        component: () => import('@/pages/HomePage.vue'),
      },
      {
        path: 'welcome',
        name: 'WelcomePage',
        component: () => import('@/pages/WelcomePage.vue'),
      },
      {
        path: 'log',
        name: 'LogPage',
        component: () => import('@/pages/LogPage.vue'),
      },
      {
        path: 'diff',
        name: 'DiffViewer',
        component: () => import('@/pages/DiffPage.vue'),
      },
      {
        path: 'resolve',
        name: 'ResolveConflict',
        component: () => import('@/components/dialogs/ResolveConflict.vue'),
      },
    ],
  },
  {
    path: '/settings',
    name: 'SettingsPage',
    component: () => import('@/pages/SettingsPage.vue'),
  },
  {
    path: '/progress-window',
    name: 'ProgressWindow',
    component: () => import('@/pages/ProgressWindowPage.vue'),
  },
]

export default createRouter({
  history: createWebHistory(),
  routes,
})
