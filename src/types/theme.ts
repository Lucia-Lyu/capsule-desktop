export type CapsuleTheme = "sage" | "charcoal" | "mist" | "blue" | "yellow";

export const capsuleThemes: { id: CapsuleTheme; label: string }[] = [
  { id: "sage", label: "Original" },
  { id: "charcoal", label: "Charcoal" },
  { id: "mist", label: "Light gray" },
  { id: "blue", label: "Soft blue" },
  { id: "yellow", label: "Soft yellow" },
];

export function isCapsuleTheme(value: string): value is CapsuleTheme {
  return capsuleThemes.some((theme) => theme.id === value);
}
