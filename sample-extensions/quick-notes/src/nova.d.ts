/// <reference types="@aspect/nova" />

// Nova global is provided by the runtime
declare const Nova: import("@aspect/nova").NovaAPI;

// Crypto is available in Deno
declare const crypto: {
  randomUUID(): string;
};
