import { sendNotification } from "@tauri-apps/plugin-notification";

/**
 * 发送系统通知
 * 在 macOS 上，系统通知会自动使用应用的图标（从应用 bundle 中获取）
 * 应用图标已在 tauri.conf.json 中配置，通知会自动显示应用图标
 * 如果看到默认图标，请重新构建应用以确保图标正确嵌入到应用 bundle 中
 * @param title 通知标题
 * @param body 通知内容
 */
export async function showSystemNotification(
  title: string,
  body: string,
): Promise<void> {
  try {
    // macOS 系统通知会自动使用应用的图标（从应用 bundle 中获取）
    // 应用图标已在 tauri.conf.json 的 bundle.icon 中配置
    await sendNotification({
      title,
      body,
    });
  } catch (err) {
    console.error("Failed to send notification:", err);
  }
}
