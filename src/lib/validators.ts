import type { RuleInput } from "./types";

export interface RuleValidationResult {
  issues: Partial<Record<keyof RuleInput, string>>;
  isValid: boolean;
}

const hostnamePattern =
  /^(localhost|(?=.{1,253}$)([a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)(?:\.([a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?))*)$/;

export function isIPv4Address(value: string) {
  const segments = value.split(".");
  if (segments.length !== 4) {
    return false;
  }

  return segments.every((segment) => {
    if (!/^\d+$/.test(segment)) {
      return false;
    }

    const parsed = Number(segment);
    return parsed >= 0 && parsed <= 255;
  });
}

export function isIPv6Address(value: string) {
  if (!value.includes(":")) {
    return false;
  }

  try {
    return new URL(`http://[${value}]`).hostname.length > 0;
  } catch {
    return false;
  }
}

export function isHostname(value: string) {
  return hostnamePattern.test(value);
}

export function isLoopbackHost(value: string) {
  return value === "127.0.0.1" || value === "::1" || value === "localhost";
}

export function needsFirewallNotice(value: string) {
  return !isLoopbackHost(value.trim());
}

export function validateRuleInput(input: RuleInput): RuleValidationResult {
  const issues: RuleValidationResult["issues"] = {};

  if (!input.name.trim()) {
    issues.name = "名称不能为空";
  }

  if (!input.listen_host.trim()) {
    issues.listen_host = "监听地址不能为空";
  } else if (!isIPv4Address(input.listen_host) && !isIPv6Address(input.listen_host)) {
    issues.listen_host = "监听地址需要是合法的 IPv4 或 IPv6";
  }

  if (!input.target_host.trim()) {
    issues.target_host = "目标地址不能为空";
  } else if (
    !isIPv4Address(input.target_host) &&
    !isIPv6Address(input.target_host) &&
    !isHostname(input.target_host)
  ) {
    issues.target_host = "目标地址需要是合法的 IP 或主机名";
  }

  if (input.listen_port < 1 || input.listen_port > 65535) {
    issues.listen_port = "监听端口必须在 1-65535 之间";
  }

  if (input.target_port < 1 || input.target_port > 65535) {
    issues.target_port = "目标端口必须在 1-65535 之间";
  }

  return {
    issues,
    isValid: Object.keys(issues).length === 0,
  };
}
