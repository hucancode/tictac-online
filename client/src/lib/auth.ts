import {
	AuthFlowType,
	CognitoIdentityProviderClient,
	InitiateAuthCommand,
	GetUserCommand,
	type GetUserCommandInput,
	type GetUserCommandOutput,
	type InitiateAuthCommandInput,
	type InitiateAuthCommandOutput,
	type GlobalSignOutCommandInput,
	GlobalSignOutCommand
} from '@aws-sdk/client-cognito-identity-provider';
import axios from 'axios';
import { writable, type Writable } from 'svelte/store';
import type { User } from '$lib/models/user';

const client = new CognitoIdentityProviderClient({
	region: 'us-east-1'
});
export const user: Writable<User | null> = writable(null);

async function getProfileInternal(shouldRetry: boolean) {
	try {
		const token = localStorage.getItem('access_token');
		if (!token) {
			throw new Error('Unauthenticated');
		}
		const input: GetUserCommandInput = {
			AccessToken: token
		};
		const command = new GetUserCommand(input);
		let response: GetUserCommandOutput = await client.send(command);
		if (response.$metadata.httpStatusCode === 401) {
			throw new Error('Unauthenticated');
		}
		if (response.$metadata.httpStatusCode !== 200) {
			shouldRetry = false;
			throw new Error('Cant get user proile');
		}
		const email =
			response.UserAttributes?.find((e) => e.Name == 'email')?.Value || response.Username;
		const avatar =
			response.UserAttributes?.find((e) => e.Name == 'avatar')?.Value ||
			`https://picsum.photos/seed/${email}/200`;
		if (!email) {
			throw new Error('Invalid user');
		}
		user.set({
			email,
			avatar
		});
	} catch (e) {
		if (shouldRetry && (await refreshToken())) {
			getProfileInternal(false);
		} else {
			throw e;
		}
	}
}

export async function getProfile(): Promise<User> {
	getProfileInternal(true);
}

export async function login(username: string, password: string): Promise<boolean> {
	const input: InitiateAuthCommandInput = {
		AuthFlow: AuthFlowType.USER_PASSWORD_AUTH,
		AuthParameters: {
			USERNAME: username,
			PASSWORD: password
		},
		ClientId: import.meta.env.VITE_COGNITO_CLIENT_ID
	};
	const response: InitiateAuthCommandOutput = await client.send(new InitiateAuthCommand(input));
	if (response.$metadata.httpStatusCode != 200) {
		console.log(response);
		return false;
	}
	const idToken = response.AuthenticationResult?.IdToken;
	const accessToken = response.AuthenticationResult?.AccessToken;
	const refreshToken = response.AuthenticationResult?.RefreshToken;
	if (!idToken || !accessToken || !refreshToken) {
		return false;
	}
	localStorage.setItem('id_token', idToken);
	localStorage.setItem('access_token', accessToken);
	localStorage.setItem('refresh_token', refreshToken);
	return true;
}

export async function logout() {
	const token = localStorage.getItem('access_token');
	localStorage.removeItem('id_token');
	localStorage.removeItem('access_token');
	localStorage.removeItem('refresh_token');
	user.set(null);
	if (!token) {
		return;
	}
	const input: GlobalSignOutCommandInput = {
		AccessToken: token
	};
	const response = await client.send(new GlobalSignOutCommand(input));
	if (response?.$metadata?.httpStatusCode !== 200) {
		console.error('error while trying to logout', response);
	}
}

export async function refreshToken(): Promise<boolean> {
	const token = localStorage.getItem('refresh_token');
	if (!token) {
		return false;
	}
	const input: InitiateAuthCommandInput = {
		AuthFlow: AuthFlowType.REFRESH_TOKEN_AUTH,
		AuthParameters: {
			REFRESH_TOKEN: token
		},
		ClientId: import.meta.env.VITE_COGNITO_CLIENT_ID
	};
	const response: InitiateAuthCommandOutput = await client.send(new InitiateAuthCommand(input));
	if (response.$metadata.httpStatusCode != 200) {
		console.log(response);
		return false;
	}
	localStorage.setItem('id_token', response.AuthenticationResult?.IdToken || '');
	localStorage.setItem('access_token', response.AuthenticationResult?.AccessToken || '');
	localStorage.setItem('refresh_token', response.AuthenticationResult?.RefreshToken || '');
	return true;
}

export function setupAuth() {
	const token = localStorage.getItem('id_token');
	console.log('setup auth with token', token);
	if (token) {
		axios.defaults.headers.common.Authorization = token;
	} else {
		delete axios.defaults.headers.common.Authorization;
	}
	axios.interceptors.response.use(
		(response) => response,
		async (error) => {
			console.log('axios error detected', error.response?.status);
			if (error.response?.status !== 401) {
				throw error;
			}
			console.log('authentication error, try refresh token');
			const success = await refreshToken();
			if (!success) {
				// redirect to login page
				// to prevent infinite loop, set a new error code
				error.response.status = 400;
				throw error;
			}
			const config = error.response.config;
			const token = localStorage.getItem('id_token');
			config.headers.Authorization = token;
			axios.defaults.headers.common.Authorization = token;
			await axios(config);
		}
	);
}
