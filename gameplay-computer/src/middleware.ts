import { defineMiddleware } from "astro:middleware";

//import { currentUser } from "./auth";
//import { db } from "./db";

export const onRequest = defineMiddleware(async (_context, next) => {
    //const user = await currentUser(context.cookies);
    return next();
});
