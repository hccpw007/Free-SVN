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
        path: 'commit',
        name: 'CommitPage',
        component: () => import('@/pages/CommitPage.vue'),
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
]

export default createRouter({
  history: createWebHistory(),
  routes,
})
