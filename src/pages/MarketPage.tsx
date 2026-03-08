import { useTranslation } from 'react-i18next'

export function MarketPage() {
  const { t } = useTranslation()

  return (
    <div className="space-y-6">
      <section className="rounded-box border border-base-300 bg-base-100 p-6">
        <h2 className="text-3xl font-semibold">{t('market.title')}</h2>
        <p className="mt-3 max-w-3xl text-sm text-base-content/65">{t('market.description')}</p>
      </section>

      <section className="rounded-box border border-dashed border-base-300 bg-base-100 p-6 text-sm text-base-content/60">
        这里会在下一阶段接入多市场搜索、安装风险提示和下载任务编排。当前保留页面壳与导航位置，
        用于确保路由、i18n、主题和桌面布局已经就绪。
      </section>
    </div>
  )
}
