# Scheduler (Rust)

- POST `/api/task/add`
  Request (JSON):
  {
    "name": "Task Name",
    "target": "example.com\n1.1.1.1/30",
    "ignore": "",
    "node": ["node-1"],
    "allNode": false,
    "scheduledTasks": false,
    "template": "<ObjectId string>",
    "duplicates": false
  }

  Response: {"code":200, "message":"Task added successfully"}

- GET `/api/node/data/online`
  Response: {"code":200, "data": {"list": ["node-1", "node-2"]}}