import { STATUS_CODES } from 'http';
import { handlePollRequest } from './poll/poll-manager';

export default {
	async fetch(request, env, ctx) {
		const url = new URL(request.url);

		if (url.pathname.startsWith('/poll')) {
			return await handlePollRequest(request, env);
		}

		if (url.pathname === '/' || !url.pathname) {
			return getResponseJson(406, 'This is an API and is not meant to be used as a webpage.');
		}

		return notFound();
	},
};

export function getResponseJson(status, detailed) {
	return new Response(JSON.stringify({
		status: status,
		message: STATUS_CODES[status],
		detailed: detailed
	}), {
		status: status,
		headers: {
			'Content-Type': 'application/json',
		}
	});
}

export function notFound() {
	return getResponseJson(404, "The requested URL was not found.");
}
