import http from 'k6/http';
import { check, sleep } from 'k6';
import { uuidv4 } from 'https://jslib.k6.io/k6-utils/1.4.0/index.js';

export const options = {
  stages: [
    { duration: '15s', target: 50 },  // Ramp-up to 50 virtual users
    { duration: '30s', target: 50 },  // Stay at 50 users (sustained load)
    { duration: '15s', target: 100 }, // Spike to 100 users
    { duration: '30s', target: 100 }, // Stay at 100 users (high stress)
    { duration: '15s', target: 0 },   // Ramp-down to 0 users
  ],
  thresholds: {
    http_req_duration: ['p(95)<150'], // 95% of requests must complete below 150ms
    http_req_failed: ['rate<0.02'],    // Less than 2% of requests should fail (excluding domain validation errors)
  },
};

const BASE_URL = 'http://localhost:3000';
const TREASURY = '00000000-0000-0000-0000-000000000000';
const USER_A = '11111111-1111-1111-1111-111111111111';
const USER_B = '22222222-2222-2222-2222-222222222222';

export default function () {
  const rand = Math.random();

  if (rand < 0.3) {
    // 1. Cash-in from Treasury to User A or B (30% of requests)
    const toAccount = Math.random() < 0.5 ? USER_A : USER_B;
    const payload = JSON.stringify({
      from_account: TREASURY,
      to_account: toAccount,
      amount: Math.floor(Math.random() * 100) + 1, // random deposit between 1 and 100
      idempotency_key: uuidv4(),
    });

    const res = http.post(`${BASE_URL}/transfers`, payload, {
      headers: { 'Content-Type': 'application/json' },
    });

    check(res, {
      'cash-in status is 201': (r) => r.status === 201,
    });

  } else if (rand < 0.6) {
    // 2. Transfer between Account A and Account B (30% of requests)
    const fromAccount = Math.random() < 0.5 ? USER_A : USER_B;
    const toAccount = fromAccount === USER_A ? USER_B : USER_A;
    const payload = JSON.stringify({
      from_account: fromAccount,
      to_account: toAccount,
      amount: Math.floor(Math.random() * 10) + 1, // random transfer between 1 and 10
      idempotency_key: uuidv4(),
    });

    const res = http.post(`${BASE_URL}/transfers`, payload, {
      headers: { 'Content-Type': 'application/json' },
    });

    // Note: a status of 422 is a valid domain validation error (insufficient funds),
    // which is not a server failure (500) but expected business logic.
    check(res, {
      'transfer status is 201 or 422': (r) => r.status === 201 || r.status === 422,
    });

  } else if (rand < 0.8) {
    // 3. Query balance (20% of requests)
    const account = Math.random() < 0.5 ? USER_A : USER_B;
    const res = http.get(`${BASE_URL}/accounts/${account}/balance`);

    check(res, {
      'balance status is 200': (r) => r.status === 200,
    });

  } else {
    // 4. Query statement / transactions (20% of requests)
    const account = Math.random() < 0.5 ? USER_A : USER_B;
    const res = http.get(`${BASE_URL}/accounts/${account}/transactions`);

    check(res, {
      'statement status is 200': (r) => r.status === 200,
    });
  }

  sleep(0.1); // Sleep for 100ms to simulate real-user latency and avoid pegging CPU completely
}
