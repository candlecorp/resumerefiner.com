export class API {
	async getUserInfo() {
		try {
			const response = await fetch('/oidc/userinfo', { method: 'GET', redirect: 'manual' });
			if (response.ok) {
				const data = await response.json();
				return data;
			}
		} catch (e) {
			console.log('Error fetching userinfo -', e);
		}
	}

	async isStargazer() {
		try {
			const response = await fetch('/stargazer', { method: 'GET', redirect: 'manual' });
			if (response.ok) {
				const data = await response.json();
				return data;
			}
		} catch (e) {
			console.log('Error fetching stargazer -', e);
		}
	}

	async confirmStar() {
		try {
			const response = await fetch('/confirm_star', { method: 'GET', redirect: 'manual' });
			if (response.ok) {
				const data = await response.json();
				return data;
			}
		} catch (e) {
			console.log('Error fetching confirm star -', e);
		}
	}

	async getDailyUsage() {
		try {
			const response = await fetch('/check_usage', { method: 'GET', redirect: 'manual' });
			if (response.ok) {
				const data = await response.json();
				return data;
			}
		} catch (e) {
			console.log('Error fetching daily usage -', e);
		}
	}
}
