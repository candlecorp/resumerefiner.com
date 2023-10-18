export interface UserInfo {
	email: string;
	sub: string;
}

export interface TokenUsage {
	entitlement_name: string;
	entitlement_value: number;
	daily_usage_value: number;
}

export interface ConfirmStar {
	status: boolean;
	message: string;
}

export interface Stargazer {
	is_stargazer: boolean;
}
