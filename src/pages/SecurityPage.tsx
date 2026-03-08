import { useTranslation } from 'react-i18next'

export function SecurityPage() {
  const { t } = useTranslation()

  return (
    <div className="space-y-6">
      <section className="rounded-box border border-base-300 bg-base-100 p-6">
        <h2 className="text-3xl font-semibold">{t('security.title')}</h2>
        <p className="mt-3 max-w-3xl text-sm text-base-content/65">{t('security.description')}</p>
      </section>

      <section className="grid gap-4 md:grid-cols-3">
        {['Safe', 'Medium', 'Blocked'].map((label) => (
          <div key={label} className="rounded-box border border-base-300 bg-base-100 p-5">
            <p className="text-sm uppercase tracking-[0.2em] text-base-content/50">{label}</p>
            <p className="mt-3 text-3xl font-semibold">0</p>
          </div>
        ))}
      </section>
    </div>
  )
}
