import { invoke } from "@tauri-apps/api/core";

/**
 * 发送系统通知
 * 通知由 Rust 后端通过 notify-rust 发送，确保文本通知与图片通知
 * 使用同一套系统原生实现和应用标识。
 * @param title 通知标题
 * @param body 通知内容
 */
export async function showSystemNotification(
  title: string,
  body: string,
): Promise<void> {
  try {
    await invoke("show_system_notification", {
      title,
      body,
    });
  } catch (err) {
    console.error("Failed to send notification:", err);
  }
}
