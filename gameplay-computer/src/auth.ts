import { z } from "zod";
import { jwtVerify, importSPKI, errors } from "jose";
import type { AstroCookies } from "astro";

const publicKey = import.meta.env.CLERK_JWT_PUBLIC_KEY;

const ClerkUser = z.object({
    clerk_id: z.string(),
    email: z.string(),
    first_name: z.string(),
    last_name: z.string(),
    username: z.string(),
});

export type ClerkUser = z.infer<typeof ClerkUser>;

export async function currentUser(
    cookies: AstroCookies,
): Promise<ClerkUser | null> {
    const cookie = cookies.get("__session");
    if (cookie && cookie.value) {
        const token = cookie.value;
        // todo perf: construct this at startup
        const key = await importSPKI(publicKey, "RS256");
        try {
            const decoded = await jwtVerify(token, key);
            const user = ClerkUser.parse(decoded.payload);
            return user;
        } catch (e) {
            if (e instanceof errors.JWTExpired) {
                // todo: refresh token
            } else {
                console.error(e);
            }
        }
    }
    return null;
}
