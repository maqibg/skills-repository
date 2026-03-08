import { useTranslation } from 'react-i18next'
import { useSettingsStore } from '../stores/use-settings-store'
import { useSkillsStore } from '../stores/use-skills-store'

export function OverviewPage() {
  const { t } = useTranslation()
  const scanSkills = useSkillsStore((state) => state.scanSkills)
  const projects = useSkillsStore((state) => state.projects)
  const settings = useSettingsStore((state) => state.settings)

  return (
    <div className="space-y-6">
      <section className="rounded-box border border-base-300 bg-base-100 p-6">
        <div className="flex items-start justify-between gap-4">
          <div>
            <p className="text-sm uppercase tracking-[0.24em] text-primary">Phase 1</p>
            <h2 className="mt-2 text-3xl font-semibold">{t('overview.title')}</h2>
            <p className="mt-3 max-w-3xl text-sm leading-6 text-base-content/65">
              {t('overview.description')}
            </p>
          </div>

          <button
            className="btn btn-primary"
            onClick={() =>
              void scanSkills({
                includeSystem: true,
                includeProjects: true,
                projectRoots: settings.scan.projectRoots,
                customRoots: settings.scan.customRoots,
              })
            }
          >
            {t('overview.scanNow')}
          </button>
        </div>
      </section>

      <section className="grid gap-6 xl:grid-cols-[1.4fr_1fr]">
        <div className="rounded-box border border-base-300 bg-base-100 p-6">
          <h3 className="text-lg font-semibold">{t('overview.recentTasks')}</h3>
          <p className="mt-2 text-sm text-base-content/60">{t('overview.recentTasksHint')}</p>
          <div className="mt-6 grid gap-3">
            <div className="rounded-box border border-dashed border-base-300 bg-base-200/60 p-4 text-sm text-base-content/60">
              当前阶段暂未接入市场安装、分发与模板注入闭环；任务中心已可承接扫描进度事件。
            </div>
          </div>
        </div>

        <div className="rounded-box border border-base-300 bg-base-100 p-6">
          <h3 className="text-lg font-semibold">{t('overview.agents')}</h3>
          <div className="mt-4 space-y-3">
            {projects.length === 0 ? (
              <div className="rounded-box border border-dashed border-base-300 bg-base-200/60 p-4 text-sm text-base-content/60">
                尚未检测到项目级配置。你可以先在设置页填写项目根路径。
              </div>
            ) : (
              projects.map((project) => (
                <div key={project.id} className="rounded-box border border-base-300 bg-base-200/60 p-4">
                  <p className="font-medium">{project.name}</p>
                  <p className="mt-1 text-xs text-base-content/55">{project.rootPath}</p>
                </div>
              ))
            )}
          </div>
        </div>
      </section>
    </div>
  )
}
