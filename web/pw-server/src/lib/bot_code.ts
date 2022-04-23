import defaultBotCode from "../assets/bot_template.txt?raw";

const BOT_CODE_KEY = "bot_code";

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
