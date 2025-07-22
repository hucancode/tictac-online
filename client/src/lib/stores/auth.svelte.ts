import { authApi, type UserProfile, type LoginData, type RegisterData } from '../api';

interface AuthState {
  user: UserProfile | null;
  token: string | null;
  loading: boolean;
  error: string | null;
}

// Create reactive state using Svelte 5 runes
let authState = $state<AuthState>({
  user: null,
  token: null,
  loading: false,
  error: null,
});

// Initialize from localStorage
function initializeAuth() {
  if (typeof window === 'undefined') return;
  
  const token = localStorage.getItem('auth_token');
  const userStr = localStorage.getItem('user');
  
  if (token && userStr) {
    try {
      const user = JSON.parse(userStr);
      authState.user = user;
      authState.token = token;
      
      // Refresh user data
      authApi.getProfile().then((freshUser) => {
        authState.user = freshUser;
        localStorage.setItem('user', JSON.stringify(freshUser));
      }).catch(() => {
        // Token might be expired
        logout();
      });
    } catch {
      logout();
    }
  }
}

export async function login(data: LoginData) {
  authState.loading = true;
  authState.error = null;
  
  try {
    const response = await authApi.login(data);
    
    localStorage.setItem('auth_token', response.token);
    localStorage.setItem('user', JSON.stringify(response.user));
    
    authState.user = response.user;
    authState.token = response.token;
    authState.loading = false;
    
    return response;
  } catch (error: any) {
    const errorMessage = error.response?.data?.error || 'Login failed';
    authState.error = errorMessage;
    authState.loading = false;
    throw error;
  }
}

export async function register(data: RegisterData) {
  authState.loading = true;
  authState.error = null;
  
  try {
    const response = await authApi.register(data);
    
    localStorage.setItem('auth_token', response.token);
    localStorage.setItem('user', JSON.stringify(response.user));
    
    authState.user = response.user;
    authState.token = response.token;
    authState.loading = false;
    
    return response;
  } catch (error: any) {
    const errorMessage = error.response?.data?.error || 'Registration failed';
    authState.error = errorMessage;
    authState.loading = false;
    throw error;
  }
}

export function logout() {
  authApi.logout();
  authState.user = null;
  authState.token = null;
  authState.loading = false;
  authState.error = null;
}

export function updateUser(user: UserProfile) {
  authState.user = user;
  localStorage.setItem('user', JSON.stringify(user));
}

// Initialize on module load
initializeAuth();

// Export getters using $derived
export const auth = {
  get user() { return authState.user; },
  get token() { return authState.token; },
  get loading() { return authState.loading; },
  get error() { return authState.error; },
  get isAuthenticated() { return !!authState.token; },
  get isAdmin() { return authState.user?.is_admin || false; }
};