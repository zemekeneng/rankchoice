import { Request, Response, NextFunction } from 'express';

export interface AuthenticatedRequest extends Request {
  apiKey?: string;
}

export function validateApiKey(req: AuthenticatedRequest, res: Response, next: NextFunction) {
  const apiKey = req.headers['x-api-key'] as string;
  const validApiKey = process.env.EMAIL_SERVICE_API_KEY;

  // In development, allow requests without API key for easier testing
  if (process.env.NODE_ENV === 'development' && !validApiKey) {
    console.warn('⚠️  No API key configured in development mode');
    req.apiKey = 'development';
    return next();
  }

  if (!validApiKey) {
    console.error('❌ EMAIL_SERVICE_API_KEY not configured');
    return res.status(500).json({
      success: false,
      error: {
        code: 'CONFIGURATION_ERROR',
        message: 'Email service API key not configured'
      }
    });
  }

  if (!apiKey) {
    return res.status(401).json({
      success: false,
      error: {
        code: 'MISSING_API_KEY',
        message: 'API key required. Provide X-API-Key header.'
      }
    });
  }

  if (apiKey !== validApiKey) {
    return res.status(403).json({
      success: false,
      error: {
        code: 'INVALID_API_KEY',
        message: 'Invalid API key provided'
      }
    });
  }

  req.apiKey = apiKey;
  next();
}