import defaultBotCode from "../assets/bot_template.txt?raw";

const BOT_CODE_KEY = "bot_code";
const BOT_NAME_KEY = "bot_name";

export function getBotCode(): string {
  let botCode = localStorage.getItem(BOT_CODE_KEY);
  if (!botCode) {
    botCode = defaultBotCode;
  }
  return botCode;
}

export function hasBotCode(): boolean {
  return !!localStorage.getItem(BOT_CODE_KEY);
}

export function saveBotCode(botCode: string) {
  localStorage.setItem(BOT_CODE_KEY, botCode);
}

export function getBotName(): string | null {
  return localStorage.getItem(BOT_NAME_KEY);
}

export function saveBotName(name: string) {
  localStorage.setItem(BOT_NAME_KEY, name);
}
