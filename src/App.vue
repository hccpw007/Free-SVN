<script setup lang="ts">
import { onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { RouterView } from 'vue-router'
import { useSettingsStore } from '@/stores/settings'

const { locale } = useI18n()
const settingsStore = useSettingsStore()

/** 根据 settings.language 字段设置 vue-i18n locale，支持 'system' 自动检测 */
function applyLanguage(lang: string) {
  if (lang === 'system') {
    const navLang = navigator.language
    // 支持 zh-CN/en/ja/ko，其余回退 en
    if (['zh-CN', 'en', 'ja', 'ko'].includes(navLang)) {
      locale.value = navLang
    } else {
      locale.value = 'en'
    }
  } else {
    locale.value = lang
  }
}

onMounted(async () => {
  await settingsStore.load()
  applyLanguage(settingsStore.language)
})
</script>

<template>
  <RouterView />
</template>
