## 部署架構 (Deployment Architecture)


```yaml
infrastructure:
  environment: "Single production environment"
  resources: "2 CPU cores, 4GB RAM"
  platform: "Docker containers on cloud VPS"
  
orchestration:
  service_management: "systemd or docker-compose"
  startup_time: "< 30 seconds cold start"
  
ci_cd:
  pipeline: "GitHub Actions"
  deployment: "Automated with health checks"
```

