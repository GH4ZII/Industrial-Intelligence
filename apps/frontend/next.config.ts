import type { NextConfig } from "next";
import path from "path";

const nextConfig: NextConfig = {
  // Keep Turbopack rooted in this app (avoids picking up lockfiles outside the repo)
  turbopack: {
    root: path.join(__dirname),
  },
};

export default nextConfig;
