import { describe, expect, it } from "vitest";
import { suggestTaskFromRoles } from "./types";

describe("suggestTaskFromRoles", () => {
  it("suggests replace-all when logo, audio and srt are present", () => {
    expect(suggestTaskFromRoles(["video", "logo", "audio", "srt"])).toBe("replace-all");
  });

  it("returns null when the role mix is ambiguous", () => {
    expect(suggestTaskFromRoles(["video", "logo", "audio"])).toBeNull();
  });
});
