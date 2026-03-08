import { useTranslation } from 'react-i18next'
import { useSkillsStore } from '../stores/use-skills-store'

export function SkillsPage() {
  const { t } = useTranslation()
  const skills = useSkillsStore((state) => state.skills)

  return (
    <div className="space-y-6">
      <section className="rounded-box border border-base-300 bg-base-100 p-6">
        <h2 className="text-3xl font-semibold">{t('skills.title')}</h2>
        <p className="mt-3 max-w-3xl text-sm text-base-content/65">{t('skills.description')}</p>
      </section>

      <section className="overflow-hidden rounded-box border border-base-300 bg-base-100">
        {skills.length === 0 ? (
          <div className="p-6 text-sm text-base-content/60">{t('skills.empty')}</div>
        ) : (
          <div className="overflow-x-auto">
            <table className="table">
              <thead>
                <tr>
                  <th>{t('common.agent')}</th>
                  <th>Name</th>
                  <th>{t('common.scope')}</th>
                  <th>{t('common.status')}</th>
                  <th>{t('common.path')}</th>
                </tr>
              </thead>
              <tbody>
                {skills.map((skill) => (
                  <tr key={skill.id}>
                    <td>{skill.agent}</td>
                    <td>{skill.name}</td>
                    <td>{skill.scope}</td>
                    <td>{skill.managed ? t('common.managed') : t('common.unmanaged')}</td>
                    <td className="max-w-[20rem] truncate font-mono text-xs">{skill.path}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </section>
    </div>
  )
}
