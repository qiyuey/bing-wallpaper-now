import { getSmallIconProps } from "../config/icons";
import { useI18n } from "../i18n/I18nContext";
import styles from "./About.module.css";
import modalStyles from "../styles/modal.module.css";
import btnStyles from "../styles/buttons.module.css";

interface AboutProps {
  onClose: () => void;
  version: string;
}

export function About({ onClose, version }: AboutProps) {
  const { t } = useI18n();

  return (
    <div className={modalStyles.overlay}>
      <div className={modalStyles.modal} style={{ maxWidth: "480px" }}>
        <div className={modalStyles.header}>
          <h2>{t("about")}</h2>
          <button onClick={onClose} className={btnStyles.btnClose}>
            ×
          </button>
        </div>

        <div className={modalStyles.body} style={{ textAlign: "center" }}>
          <div className={styles.titleWrapper}>
            <h1 className={styles.title}>{t("aboutTitle")}</h1>
            <p className={styles.version}>
              {t("aboutVersion")} {version}
            </p>
          </div>

          <div className={styles.infoBox}>
            <p className={styles.description}>{t("aboutDescription")}</p>
          </div>

          <div className={styles.githubSection}>
            <a
              href="https://github.com/qiyuey/bing-wallpaper-now"
              target="_blank"
              rel="noopener noreferrer"
              className={styles.githubLink}
            >
              <svg {...getSmallIconProps()} fill="currentColor">
                <path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.013 8.013 0 0016 8c0-4.42-3.58-8-8-8z" />
              </svg>
              {t("aboutGitHub")}
            </a>
          </div>

          <p className={styles.copyright}>{t("aboutCopyright")}</p>
        </div>
      </div>
    </div>
  );
}
