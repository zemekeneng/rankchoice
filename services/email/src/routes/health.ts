import { Router, Request, Response } from 'express';
import { testEmailConnection } from '../config/email';

const router = Router();

// GET /health - Basic health check
router.get('/', (req: Request, res: Response) => {
  res.json({
    status: 'ok',
    service: 'rankchoice-email-service',
    version: process.env.npm_package_version || '1.0.0',
    timestamp: new Date().toISOString(),
    uptime: process.uptime(),
    environment: process.env.NODE_ENV || 'development'
  });
});

// GET /health/detailed - Detailed health check including email connectivity
router.get('/detailed', async (req: Request, res: Response) => {
  const healthData = {
    status: 'ok',
    service: 'rankchoice-email-service',
    version: process.env.npm_package_version || '1.0.0',
    timestamp: new Date().toISOString(),
    uptime: process.uptime(),
    environment: process.env.NODE_ENV || 'development',
    checks: {
      email: {
        status: 'unknown',
        lastChecked: new Date().toISOString()
      },
      memory: {
        used: process.memoryUsage().heapUsed,
        total: process.memoryUsage().heapTotal,
        percentage: Math.round((process.memoryUsage().heapUsed / process.memoryUsage().heapTotal) * 100)
      }
    }
  };

  try {
    const emailConnectionOk = await testEmailConnection();
    healthData.checks.email.status = emailConnectionOk ? 'ok' : 'error';
    
    if (!emailConnectionOk) {
      healthData.status = 'degraded';
    }
  } catch (error) {
    healthData.checks.email.status = 'error';
    healthData.status = 'degraded';
  }

  const statusCode = healthData.status === 'ok' ? 200 : 503;
  res.status(statusCode).json(healthData);
});

export { router as healthRouter };