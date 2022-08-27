import { writable } from "svelte/store";

const MAX_MATCHES = 100;

function createMatchHistory() {
  const { subscribe, update } = writable([]);

  function pushMatch(match: object) {
    update((matches) => {
      if (matches.length == MAX_MATCHES) {
        matches.pop();
      }
      matches.unshift(match);

      return matches;
    });
  }

  return {
    subscribe,
    pushMatch,
  };
}

export const matchHistory = createMatchHistory();
export const selectedOpponent = writable(null);
export const selectedMap = writable(null);
