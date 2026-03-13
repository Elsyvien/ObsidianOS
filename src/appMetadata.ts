import { isTauri } from "@tauri-apps/api/core";
import type { AppMetadata } from "./types";

const DEFAULT_PRODUCT_NAME = __APP_PRODUCT_NAME__;
const DEFAULT_VERSION = __APP_VERSION__;
const DEFAULT_VERSION_DISPLAY = __APP_VERSION_DISPLAY__;
const DEFAULT_CODENAME = __APP_CODENAME__;
const DEFAULT_COPYRIGHT = __APP_COPYRIGHT__;

export const DEFAULT_APP_METADATA: AppMetadata = {
  productName: DEFAULT_PRODUCT_NAME,
  version: DEFAULT_VERSION,
  versionDisplay: DEFAULT_VERSION_DISPLAY,
  codename: DEFAULT_CODENAME,
  copyright: DEFAULT_COPYRIGHT,
};

export async function resolveAppMetadata(): Promise<AppMetadata> {
  if (!isTauri()) {
    return DEFAULT_APP_METADATA;
  }

  try {
    const [{ getName, getVersion }] = await Promise.all([import("@tauri-apps/api/app")]);
    const [productName, version] = await Promise.all([getName(), getVersion()]);

    return {
      ...DEFAULT_APP_METADATA,
      productName: productName || DEFAULT_APP_METADATA.productName,
      version: version || DEFAULT_APP_METADATA.version,
      versionDisplay: formatVersionDisplay(version || DEFAULT_APP_METADATA.version),
    };
  } catch {
    return DEFAULT_APP_METADATA;
  }
}

function formatVersionDisplay(version: string) {
  return `V.${version.replace(/(^|[-.])([a-z])/g, (_, prefix, letter) => `${prefix}${letter.toUpperCase()}`)}`;
}
