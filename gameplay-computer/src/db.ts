import { drizzle } from "drizzle-orm/planetscale-serverless";
import { connect } from "@planetscale/database";

const connection = connect({
    host: import.meta.env.DATABASE_HOST,
    username: import.meta.env.DATABASE_USERNAME,
    password: import.meta.env.DATABASE_PASSWORD,
});
 
export const db = drizzle(connection);
