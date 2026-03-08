import i18n from 'i18next'
import LanguageDetector from 'i18next-browser-languagedetector'
import { initReactI18next } from 'react-i18next'
import zhCN from '../../locales/zh-CN/common.json'
import enUS from '../../locales/en-US/common.json'
import jaJP from '../../locales/ja-JP/common.json'

void i18n
  .use(LanguageDetector)
  .use(initReactI18next)
  .init({
    resources: {
      'zh-CN': { common: zhCN },
      'en-US': { common: enUS },
      'ja-JP': { common: jaJP },
    },
    lng: 'en-US',
    fallbackLng: 'en-US',
    supportedLngs: ['zh-CN', 'en-US', 'ja-JP'],
    ns: ['common'],
    defaultNS: 'common',
    interpolation: {
      escapeValue: false,
    },
    detection: {
      order: ['navigator'],
      caches: [],
    },
  })

export default i18n
