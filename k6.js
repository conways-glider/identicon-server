import http from 'k6/http';
import { sleep } from 'k6';


export default function () {

  http.get('http://127.0.0.1:8080/test_large?scale=420');

  // sleep(1);
}
