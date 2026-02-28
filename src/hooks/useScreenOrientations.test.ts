import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { useScreenOrientations } from "./useScreenOrientations";

// Mock Tauri API
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

describe("useScreenOrientations", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("应该正确获取屏幕方向信息", async () => {
    const mockOrientations = [
      {
        screen_index: 0,
        is_portrait: false,
        width: 1920,
        height: 1080,
      },
      {
        screen_index: 1,
        is_portrait: true,
        width: 1080,
        height: 1920,
      },
    ];

    vi.mocked(invoke).mockResolvedValue(mockOrientations);

    const { result } = renderHook(() => useScreenOrientations());

    // 初始状态应该是加载中
    expect(result.current.loading).toBe(true);
    expect(result.current.orientations).toEqual([]);

    // 等待加载完成
    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    // 验证结果
    expect(result.current.orientations).toEqual(mockOrientations);
    expect(result.current.error).toBeNull();
    expect(invoke).toHaveBeenCalledWith("get_screen_orientations");
  });

  it("应该正确处理错误", async () => {
    const error = new Error("获取屏幕方向失败");
    vi.mocked(invoke).mockRejectedValue(error);

    const { result } = renderHook(() => useScreenOrientations());

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    expect(result.current.error).toBe("获取屏幕方向失败");
    expect(result.current.orientations).toEqual([]);
  });

  it("isPortrait 应该正确判断屏幕方向", async () => {
    const mockOrientations = [
      {
        screen_index: 0,
        is_portrait: false,
        width: 1920,
        height: 1080,
      },
      {
        screen_index: 1,
        is_portrait: true,
        width: 1080,
        height: 1920,
      },
    ];

    vi.mocked(invoke).mockResolvedValue(mockOrientations);

    const { result } = renderHook(() => useScreenOrientations());

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    // 测试主屏幕（索引 0）是横屏
    expect(result.current.isPortrait(0)).toBe(false);

    // 测试第二个屏幕（索引 1）是竖屏
    expect(result.current.isPortrait(1)).toBe(true);

    // 测试不存在的屏幕索引
    expect(result.current.isPortrait(2)).toBe(false);
  });

  it("getScreenSize 应该正确返回屏幕尺寸", async () => {
    const mockOrientations = [
      {
        screen_index: 0,
        is_portrait: false,
        width: 1920,
        height: 1080,
      },
    ];

    vi.mocked(invoke).mockResolvedValue(mockOrientations);

    const { result } = renderHook(() => useScreenOrientations());

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    const size = result.current.getScreenSize(0);
    expect(size).toEqual({ width: 1920, height: 1080 });

    // 测试不存在的屏幕索引
    const invalidSize = result.current.getScreenSize(1);
    expect(invalidSize).toBeNull();
  });

  it("fetchOrientations 应该可以手动调用", async () => {
    const mockOrientations = [
      {
        screen_index: 0,
        is_portrait: false,
        width: 1920,
        height: 1080,
      },
    ];

    vi.mocked(invoke).mockResolvedValue(mockOrientations);

    const { result } = renderHook(() => useScreenOrientations());

    // 等待初始加载完成
    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    // 清空 mock 调用记录
    vi.clearAllMocks();

    // 手动调用 fetchOrientations
    await result.current.fetchOrientations();

    // 验证再次调用了 invoke
    expect(invoke).toHaveBeenCalledWith("get_screen_orientations");
  });
});
