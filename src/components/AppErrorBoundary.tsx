import { Component, ErrorInfo, ReactNode } from "react";
import styles from "./AppErrorBoundary.module.css";

interface AppErrorBoundaryProps {
  children: ReactNode;
  onError?: (error: Error, info: ErrorInfo) => void;
}

interface AppErrorBoundaryState {
  error: Error | null;
}

export class AppErrorBoundary extends Component<
  AppErrorBoundaryProps,
  AppErrorBoundaryState
> {
  state: AppErrorBoundaryState = {
    error: null,
  };

  static getDerivedStateFromError(error: Error): AppErrorBoundaryState {
    return { error };
  }

  componentDidCatch(error: Error, info: ErrorInfo) {
    this.props.onError?.(error, info);
  }

  render() {
    if (!this.state.error) {
      return this.props.children;
    }

    return (
      <main className={styles.errorScreen} role="alert">
        <section className={styles.errorPanel}>
          <h1 className={styles.errorTitle}>应用启动失败 / Startup failed</h1>
          <p className={styles.errorBody}>
            请退出并重新打开应用。错误详情已经写入应用日志。
          </p>
          <pre className={styles.errorMessage}>{this.state.error.message}</pre>
        </section>
      </main>
    );
  }
}
