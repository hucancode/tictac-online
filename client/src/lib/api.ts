import axios from 'axios';
import { getApiUrl } from './config';

// Create axios instance with default config
const api = axios.create({
  baseURL: `${getApiUrl()}/api`,
  headers: {
    'Content-Type': 'application/json',
  },
  withCredentials: true, // Required for CORS with credentials
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

export interface LeaderboardResponse {
  entries: LeaderboardEntry[];
  total: number;
  page: number;
  limit: number;
  total_pages: number;
}

export interface MatchHistoryItem {
  id: string;
  opponent_id: string;
  opponent_name: string;
  opponent_elo: number;
  result: 'win' | 'loss' | 'draw';
  my_elo_before: number;
  my_elo_after: number;
  opponent_elo_before: number;
  opponent_elo_after: number;
  elo_change: number;
  created_at: string;
  ended_at?: string;
}

export interface MatchHistoryResponse {
  matches: MatchHistoryItem[];
  total: number;
  page: number;
  limit: number;
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
  getLeaderboard: async (page = 1, limit = 20): Promise<LeaderboardResponse> => {
    const response = await api.get<LeaderboardResponse>('/leaderboard', {
      params: { page, limit },
    });
    return response.data;
  },

  getTopPlayers: async (): Promise<LeaderboardEntry[]> => {
    const response = await api.get<LeaderboardEntry[]>('/leaderboard/top');
    return response.data;
  },

  getPlayerRank: async (userId: string): Promise<{ rank: number | null; message?: string }> => {
    const response = await api.get(`/leaderboard/rank/${userId}`);
    return response.data;
  },
};

export const gamesApi = {
  getMatchHistory: async (page = 1, limit = 20): Promise<MatchHistoryResponse> => {
    const response = await api.get<MatchHistoryResponse>('/games/history', {
      params: { page, limit },
    });
    return response.data;
  },

  getGameDetails: async (gameId: string) => {
    const response = await api.get(`/games/${gameId}`);
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