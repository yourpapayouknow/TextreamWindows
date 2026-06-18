import type { WordItem } from "./types";

export function isCjk(char: string) {
  const cp = char.codePointAt(0) ?? 0;
  return (
    (cp >= 0x4e00 && cp <= 0x9fff) ||
    (cp >= 0x3400 && cp <= 0x4dbf) ||
    (cp >= 0x20000 && cp <= 0x2a6df) ||
    (cp >= 0x2a700 && cp <= 0x2b73f) ||
    (cp >= 0x2b740 && cp <= 0x2b81f) ||
    (cp >= 0x2b820 && cp <= 0x2ceaf) ||
    (cp >= 0xf900 && cp <= 0xfaff) ||
    (cp >= 0x2f800 && cp <= 0x2fa1f) ||
    (cp >= 0x3040 && cp <= 0x309f) ||
    (cp >= 0x30a0 && cp <= 0x30ff) ||
    (cp >= 0xac00 && cp <= 0xd7af)
  );
}

export function splitTextIntoWords(text: string) {
  const tokens = text.split(/\s+/).filter(Boolean);
  const result: string[] = [];
  for (const token of tokens) {
    if (![...token].some(isCjk)) {
      result.push(token);
      continue;
    }
    let buffer = "";
    for (const char of [...token]) {
      if (isCjk(char)) {
        if (buffer) {
          result.push(buffer);
          buffer = "";
        }
        result.push(char);
      } else {
        buffer += char;
      }
    }
    if (buffer) result.push(buffer);
  }
  return result;
}

export function isAnnotationWord(word: string) {
  if (word.startsWith("[") && word.endsWith("]")) return true;
  return !/[A-Za-z0-9\p{L}\p{N}]/u.test(word);
}

export function buildWordItems(words: string[]): WordItem[] {
  let offset = 0;
  return words.map((word, id) => {
    const item = {
      id,
      word,
      charOffset: offset,
      isAnnotation: isAnnotationWord(word),
    };
    offset += [...word].length + 1;
    return item;
  });
}

export function charOffsetForWordProgress(words: string[], progress: number) {
  const whole = Math.max(0, Math.floor(progress));
  const frac = progress - whole;
  let offset = 0;
  for (let i = 0; i < Math.min(whole, words.length); i += 1) {
    offset += [...words[i]].length + 1;
  }
  if (whole < words.length) {
    offset += Math.floor([...words[whole]].length * Math.min(1, Math.max(0, frac)));
  }
  return Math.min(offset, words.join(" ").length);
}

