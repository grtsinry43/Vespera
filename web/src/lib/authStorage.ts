type AuthTokens = {
  accessToken: string | null;
  refreshToken: string | null;
};

let memoryTokens: AuthTokens = {
  accessToken: null,
  refreshToken: null,
};

function getStorage(): Storage | null {
  if (typeof window === 'undefined') {
    return null;
  }

  try {
    return window.sessionStorage;
  } catch {
    return null;
  }
}

export const authStorage = {
  getAccessToken(): string | null {
    const storage = getStorage();
    return storage?.getItem('token') ?? memoryTokens.accessToken;
  },

  getRefreshToken(): string | null {
    const storage = getStorage();
    return storage?.getItem('refresh_token') ?? memoryTokens.refreshToken;
  },

  setTokens(accessToken: string, refreshToken: string) {
    memoryTokens = { accessToken, refreshToken };
    const storage = getStorage();
    storage?.setItem('token', accessToken);
    storage?.setItem('refresh_token', refreshToken);
  },

  setAccessToken(accessToken: string) {
    memoryTokens.accessToken = accessToken;
    const storage = getStorage();
    storage?.setItem('token', accessToken);
  },

  clear() {
    memoryTokens = { accessToken: null, refreshToken: null };
    const storage = getStorage();
    storage?.removeItem('token');
    storage?.removeItem('refresh_token');
  },
};
