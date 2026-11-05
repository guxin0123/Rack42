import { createRouter, createWebHashHistory } from 'vue-router'

import AuditView from '@/pages/AuditView.vue'
import CredentialsView from '@/pages/CredentialsView.vue'
import DashboardView from '@/pages/DashboardView.vue'
import DevicesView from '@/pages/DevicesView.vue'
import RackDetailView from '@/pages/RackDetailView.vue'
import RacksView from '@/pages/RacksView.vue'
import SettingsView from '@/pages/SettingsView.vue'
import TopologyView from '@/pages/TopologyView.vue'

const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    {
      path: '/',
      name: 'dashboard',
      component: DashboardView,
      meta: { title: '总览' },
    },
    {
      path: '/racks',
      name: 'racks',
      component: RacksView,
      meta: { title: '机柜' },
    },
    {
      path: '/racks/:id',
      name: 'rack-detail',
      component: RackDetailView,
      meta: { title: '机柜详情' },
    },
    {
      path: '/devices',
      name: 'devices',
      component: DevicesView,
      meta: { title: '设备' },
    },
    {
      path: '/topology',
      name: 'topology',
      component: TopologyView,
      meta: { title: '拓扑' },
    },
    {
      path: '/credentials',
      name: 'credentials',
      component: CredentialsView,
      meta: { title: '凭据' },
    },
    {
      path: '/audit',
      name: 'audit',
      component: AuditView,
      meta: { title: '审计' },
    },
    {
      path: '/settings',
      name: 'settings',
      component: SettingsView,
      meta: { title: '设置' },
    },
    {
      path: '/:pathMatch(.*)*',
      redirect: '/',
    },
  ],
  scrollBehavior: () => ({ top: 0 }),
})

router.afterEach((to) => {
  const title = to.meta.title as string | undefined
  document.title = title ? `${title} · Rack42` : 'Rack42'
})

export default router
