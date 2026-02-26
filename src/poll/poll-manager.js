/*
* Incredibly exploitable implementation letting basically anyone flood the database as much as they want.
* But I trust that anyone reading this codebase is invested enough in my personal projects not to ruin it.
*
* If you feel the need to exploit this though, seriously? Why would you want to exploit this API I'm writing for fun
* and intentionally making open source on GitHub so others can learn from my (subpar) code?
*/

import { getResponseJson, notFound } from '../index';
import current from './current.json';

export async function handlePollRequest(request, env) {
	const url = new URL(request.url);
	const pathname = url.pathname.replace('/poll', '');

	switch (pathname) {
		case '/current':
			return new Response(JSON.stringify(current), {
				status: 200,
				headers: {
					'Content-Type': 'application/json',
				}
			});

		case '/upload':
			return await handlePollUpload(request, env);

		case '/fetch':
			const result = env.DB.prepare('SELECT * FROM Polls').all();
			const json = {};

			for (const row of result.results) {
				try {
					const parsed = JSON.parse(row.JsonData);
					const pollName = row.PollName;
					json[pollName] = parsed;
				} catch {
					// Ignored
				}
			}

			return new Response(JSON.stringify(json), {
				status: 200,
				headers: {
					'Content-Type': 'application/json',
				}
			});
	}

	return notFound();
}

async function handlePollUpload(request, env) {
	return getResponseJson(200, "AllGood", "Currently in progress...");
}
