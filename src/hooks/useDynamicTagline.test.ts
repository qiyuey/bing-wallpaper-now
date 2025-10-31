import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { renderHook } from "@testing-library/react";
import { useDynamicTagline } from "./useDynamicTagline";
import * as taglinesModule from "../config/taglines";
import * as translationsModule from "../i18n/translations";

vi.mock("../config/taglines");
vi.mock("../i18n/translations");

describe("useDynamicTagline", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.useFakeTimers();

    // Mock getCurrentTagline
    vi.mocked(taglinesModule.getCurrentTagline).mockReturnValue(
      "测试标语 - 当前时间",
    );

    // Mock getDailyTagline
    vi.mocked(taglinesModule.getDailyTagline).mockReturnValue(
      "测试标语 - 每日",
    );

    // Mock getAllTaglines
    vi.mocked(taglinesModule.getAllTaglines).mockImplementation((lang) => {
      if (lang === "en-US") {
        return [
          "Not all those who wander are lost.",
          "We are all in the gutter, but some of us are looking at the stars.",
          "Stay hungry, stay foolish.",
          "To strive, to seek, to find, and not to yield.",
          "The only journey is the one within.",
          "Do not go gentle into that good night.",
          "The future belongs to those who believe in the beauty of their dreams.",
          "Perhaps all the dragons in our lives are princesses who are only waiting to see us act, just once, with beauty and courage.",
          "And the end of all our exploring will be to arrive where we started and to know the place for the first time.",
          "Not I, nor anyone else can travel that road for you. You must travel it by yourself.",
        ];
      }
      return [
        "哪怕前路渺茫，也要让心中有光。",
        "世界以痛吻我，要我报之以歌。",
        "不要因为走得太远，而忘了当初为什么出发。",
        "生活不止眼前的苟且，还有诗和远方。",
        "即使被世界辜负，也不要辜负自己的热爱。",
        "黑夜给了我黑色的眼睛，我却用它寻找光明。",
        "凡心所向，素履以往。生如逆旅，一苇以航。",
        "我不能选择怎么生，怎么死，但我能决定怎么爱，怎么活。",
        "生活不能等待别人来安排，要自己去争取和奋斗。",
        "山川是不卷收的文章，日月为你掌灯伴读。",
      ];
    });

    // Mock detectSystemLanguage
    vi.mocked(translationsModule.detectSystemLanguage).mockReturnValue("zh-CN");
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("should return time-based tagline by default", () => {
    const { result } = renderHook(() => useDynamicTagline());

    expect(result.current).toBe("测试标语 - 当前时间");
    expect(taglinesModule.getCurrentTagline).toHaveBeenCalledWith(
      undefined,
      "zh-CN",
    );
  });

  it("should return time-based tagline when mode is 'time-based'", () => {
    const { result } = renderHook(() => useDynamicTagline("time-based", 60000));

    expect(result.current).toBe("测试标语 - 当前时间");
    expect(taglinesModule.getCurrentTagline).toHaveBeenCalledWith(
      undefined,
      "zh-CN",
    );
  });

  it("should return daily tagline when mode is 'daily'", () => {
    const { result } = renderHook(() => useDynamicTagline("daily", 60000));

    expect(result.current).toBe("测试标语 - 每日");
    expect(taglinesModule.getDailyTagline).toHaveBeenCalledWith("zh-CN");
  });

  it("should return random tagline when mode is 'random'", () => {
    const { result } = renderHook(() => useDynamicTagline("random", 60000));

    // Random mode should return one of the predefined taglines
    const possibleTaglines = vi.mocked(taglinesModule.getAllTaglines)("zh-CN");
    expect(possibleTaglines).toContain(result.current);
  });

  it("should update tagline when language changes", () => {
    const { result, rerender } = renderHook<
      string,
      { lang: "zh-CN" | "en-US" }
    >(({ lang }) => useDynamicTagline("time-based", 60000, lang), {
      initialProps: { lang: "zh-CN" },
    });

    expect(result.current).toBe("测试标语 - 当前时间");

    vi.mocked(taglinesModule.getCurrentTagline).mockReturnValue(
      "Test Tagline - Current Time",
    );

    rerender({ lang: "en-US" });

    // Should call with new language
    expect(taglinesModule.getCurrentTagline).toHaveBeenCalledWith(
      undefined,
      "en-US",
    );
  });

  it("should update tagline periodically for time-based mode", () => {
    vi.clearAllMocks();
    renderHook(() => useDynamicTagline("time-based", 1000));

    const initialCallCount = vi.mocked(taglinesModule.getCurrentTagline).mock
      .calls.length;

    // Fast-forward time
    vi.advanceTimersByTime(1000);

    // Should have been called again
    expect(
      vi.mocked(taglinesModule.getCurrentTagline).mock.calls.length,
    ).toBeGreaterThan(initialCallCount);
  });

  it("should not update tagline periodically for daily mode", () => {
    vi.clearAllMocks();
    const { result } = renderHook(() => useDynamicTagline("daily", 1000));

    const initialTagline = result.current;
    const initialCallCount = vi.mocked(taglinesModule.getDailyTagline).mock
      .calls.length;

    // Fast-forward time
    vi.advanceTimersByTime(1000);

    // Should not update (daily mode doesn't use interval)
    expect(result.current).toBe(initialTagline);
    expect(vi.mocked(taglinesModule.getDailyTagline).mock.calls.length).toBe(
      initialCallCount,
    );
  });

  it("should update tagline periodically for random mode", () => {
    vi.clearAllMocks();
    const { result } = renderHook(() => useDynamicTagline("random", 1000));

    // Fast-forward time
    vi.advanceTimersByTime(1000);

    // Tagline may change (random) or stay the same, but should still be valid
    const possibleTaglines = vi.mocked(taglinesModule.getAllTaglines)("zh-CN");
    expect(possibleTaglines).toContain(result.current);
  });

  it("should use provided language instead of system language", () => {
    renderHook(() => useDynamicTagline("time-based", 60000, "en-US"));

    expect(taglinesModule.getCurrentTagline).toHaveBeenCalledWith(
      undefined,
      "en-US",
    );
    expect(translationsModule.detectSystemLanguage).not.toHaveBeenCalled();
  });

  it("should use system language when lang is not provided", () => {
    renderHook(() => useDynamicTagline("time-based", 60000));

    expect(translationsModule.detectSystemLanguage).toHaveBeenCalled();
    expect(taglinesModule.getCurrentTagline).toHaveBeenCalledWith(
      undefined,
      "zh-CN",
    );
  });

  it("should return English taglines for en-US language in random mode", () => {
    const { result } = renderHook(() =>
      useDynamicTagline("random", 60000, "en-US"),
    );

    const possibleTaglines = vi.mocked(taglinesModule.getAllTaglines)("en-US");
    expect(possibleTaglines).toContain(result.current);
  });

  it("should clean up interval on unmount", () => {
    vi.clearAllMocks();
    const { unmount } = renderHook(() => useDynamicTagline("time-based", 1000));

    // Initial call
    const initialCallCount = vi.mocked(taglinesModule.getCurrentTagline).mock
      .calls.length;

    unmount();

    // Fast-forward time after unmount
    vi.advanceTimersByTime(1000);

    // Should not be called again after unmount
    expect(vi.mocked(taglinesModule.getCurrentTagline).mock.calls.length).toBe(
      initialCallCount,
    );
  });

  it("should update tagline when mode changes", () => {
    const { result, rerender } = renderHook<
      string,
      { mode: "time-based" | "daily" }
    >(({ mode }) => useDynamicTagline(mode, 60000), {
      initialProps: { mode: "time-based" },
    });

    expect(result.current).toBe("测试标语 - 当前时间");

    rerender({ mode: "daily" });

    // Should switch to daily tagline
    expect(result.current).toBe("测试标语 - 每日");
    expect(taglinesModule.getDailyTagline).toHaveBeenCalled();
  });

  it("should handle custom update interval", () => {
    vi.clearAllMocks();
    const { unmount } = renderHook(() => useDynamicTagline("time-based", 5000));

    // Initial call happens immediately
    const initialCallCount = vi.mocked(taglinesModule.getCurrentTagline).mock
      .calls.length;
    expect(initialCallCount).toBeGreaterThan(0);

    // Fast-forward 5000ms - should trigger interval call
    vi.advanceTimersByTime(5000);

    // Should have been called at least once more
    expect(
      vi.mocked(taglinesModule.getCurrentTagline).mock.calls.length,
    ).toBeGreaterThan(initialCallCount);

    unmount();
  });
});
