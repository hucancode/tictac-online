import axios from 'axios';
import { PUBLIC_API_URL } from '$env/static/public';

const API_URL = PUBLIC_API_URL ? `${PUBLIC_API_URL}/api` : 'http://localhost:8080/api';

// Create axios instance with default config
const api = axios.create({
  baseURL: API_URL,
  headers: {
    'Content-Type': 'application/json',
  },
});

// Add auth token to requests
api.interceptors.request.use((config) => {
  const token = localStorage.getItem('auth_token');
  if (token) {
    config.headers.Authorization = `Bearer ${token}`;
  }
  return config;
});

// Handle auth errors
api.interceptors.response.use(
  (response) => response,
  (error) => {
    if (error.response?.status === 401) {
      localStorage.removeItem('auth_token');
      localStorage.removeItem('user');
      window.location.href = '/';
    }
    return Promise.reject(error);
  }
);

export interface RegisterData {
  email: string;
  username: string;
  password: string;
}

export interface LoginData {
  email: string;
  password: string;
}

export interface UserProfile {
  id: string;
  email: string;
  username: string;
  profile_picture?: string;
  elo: number;
  games_played: number;
  games_won: number;
  win_rate: number;
}

export interface LoginResponse {
  token: string;
  user: UserProfile;
}

export interface LeaderboardEntry {
  rank: number;
  user_id: string;
  username: string;
  elo: number;
  games_played: number;
  win_rate: number;
  profile_picture?: string;
}

export const authApi = {
  register: async (data: RegisterData): Promise<LoginResponse> => {
    const response = await api.post<LoginResponse>('/auth/register', data);
    return response.data;
  },

  login: async (data: LoginData): Promise<LoginResponse> => {
    const response = await api.post<LoginResponse>('/auth/login', data);
    return response.data;
  },

  getProfile: async (): Promise<UserProfile> => {
    const response = await api.get<UserProfile>('/auth/me');
    return response.data;
  },

  logout: () => {
    localStorage.removeItem('auth_token');
    localStorage.removeItem('user');
  },
};

export const userApi = {
  getProfile: async (userId: string): Promise<UserProfile> => {
    const response = await api.get<UserProfile>(`/users/${userId}`);
    return response.data;
  },

  updateProfile: async (data: { username?: string }): Promise<UserProfile> => {
    const response = await api.put<UserProfile>('/users/profile', data);
    return response.data;
  },

  uploadProfilePicture: async (file: File): Promise<void> => {
    const formData = new FormData();
    formData.append('image', file);
    await api.post('/users/profile/picture', formData, {
      headers: { 'Content-Type': 'multipart/form-data' },
    });
  },
};

export const leaderboardApi = {
  getLeaderboard: async (limit = 20, offset = 0): Promise<LeaderboardEntry[]> => {
    const response = await api.get<LeaderboardEntry[]>('/leaderboard', {
      params: { limit, offset },
    });
    return response.data;
  },

  getTopPlayers: async (): Promise<LeaderboardEntry[]> => {
    const response = await api.get<LeaderboardEntry[]>('/leaderboard/top');
    return response.data;
  },
};

export const adminApi = {
  listUsers: async (limit = 50, offset = 0, search?: string) => {
    const response = await api.get('/admin/users', {
      params: { limit, offset, search },
    });
    return response.data;
  },

  updateUser: async (userId: string, data: any) => {
    const response = await api.put(`/admin/users/${userId}`, data);
    return response.data;
  },

  deleteUser: async (userId: string) => {
    const response = await api.delete(`/admin/users/${userId}`);
    return response.data;
  },

  getStats: async () => {
    const response = await api.get('/admin/stats');
    return response.data;
  },
};

export default api;