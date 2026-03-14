type AuthTokens = {
  accessToken: string | null;
};

let memoryTokens: AuthTokens = {
  accessToken: null,
};

export const authStorage = {
  getAccessToken(): string | null {
    return memoryTokens.accessToken;
  },

  setTokens(accessToken: string) {
    memoryTokens = { accessToken };
  },

  setAccessToken(accessToken: string) {
    memoryTokens.accessToken = accessToken;
  },

  clear() {
    memoryTokens = { accessToken: null };
  },
};
