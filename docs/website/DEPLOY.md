# Deploying the Kite Website

The website is a static site (no build step) deployed to Cloudflare Pages
via the native GitHub integration at `kite.kafkade.com`.

## Setup (one-time)

### 1. Create the Cloudflare Pages project

1. Go to [Cloudflare Dashboard](https://dash.cloudflare.com) → **Workers & Pages** → **Create**
2. Select the **Pages** tab → **Connect to Git**
3. Authorize GitHub and select the `kafkade/kite` repository
4. Configure the build settings:

   | Setting              | Value            |
   |----------------------|------------------|
   | **Project name**     | `kite`           |
   | **Production branch**| `main`           |
   | **Build command**    | *(leave empty)*  |
   | **Build output directory** | `docs/website` |
   | **Root directory**   | `/`              |

5. Click **Save and Deploy**

> Since this is a static site with no build step, leave the build command
> empty. Cloudflare will serve the contents of `docs/website/` directly.

### 2. Configure the custom domain

1. In the Cloudflare Pages project → **Custom domains** → **Set up a custom domain**
2. Enter `kite.kafkade.com`
3. Cloudflare will automatically add a CNAME record to your `kafkade.com` DNS zone:

   ```
   CNAME  kite  →  kite.pages.dev
   ```

4. Wait for SSL certificate provisioning (usually under 5 minutes)
5. Verify: `https://kite.kafkade.com` should serve the website

### 3. Verify deploy triggers

Once connected, Cloudflare automatically:
- Deploys on every push to `main` that touches `docs/website/`
- Creates **preview deployments** for pull requests
- Provides deploy URLs in PR comments

No GitHub Actions workflow is needed — Cloudflare handles CI/CD natively.

## Files

```
docs/website/
├── index.html     # Main page
├── style.css      # Stylesheet
├── logo.svg       # Brand logo
└── _headers       # Cloudflare caching & security headers
```

## Headers

The `_headers` file configures:
- **Aggressive caching** for CSS and SVG (1 year, immutable)
- **Short caching** for HTML (1 hour)
- **Security headers**: `X-Content-Type-Options`, `X-Frame-Options`,
  `Referrer-Policy`, `Permissions-Policy`

## Local preview

```bash
npx serve docs/website
```
