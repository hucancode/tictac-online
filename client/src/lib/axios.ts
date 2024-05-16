import axios from 'axios';
import { refreshToken } from './auth';

const client = axios.create({
	baseURL: import.meta.env.VITE_API_URL
});
const clientWithAuth = axios.create({
	baseURL: import.meta.env.VITE_API_URL
});

clientWithAuth.interceptors.request.use(
	async (config) => {
		const token = localStorage.getItem('id_token');
		if (token) {
			config.headers.Authorization = token;
		}
		return config;
	},
	(error) => {
		throw error;
	}
);
clientWithAuth.interceptors.response.use(
	(response) => response,
	async (error) => {
		if (error.response?.status !== 401) {
			throw error;
		}
		// console.warn('authentication error, try refresh token');
		const success = await refreshToken();
		if (!success) {
			// redirect to login page
			// error.response.status = 400;
			throw error;
		}
		const config = error.response.config;
		const token = localStorage.getItem('id_token');
		config.headers.Authorization = token;
		await axios(config);
	}
);

export { client, clientWithAuth };
