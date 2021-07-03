import * as functions from "firebase-functions";
import * as admin from "firebase-admin";
import * as express from "express";
import * as cors from "cors";
import { BraintreeGateway, Environment } from "braintree";
import { buildCustomerRecord } from "./records";

// Initialize firebase inorder to access its services
admin.initializeApp(functions.config().firebase);

// Initialize express server
const app = express();
const main = express();

// const corsOptions = {
//   origin: ["http://localhost", "https://phosphoricons.com"],

//   allowedHeaders: [
//     "Content-Type",
//     "Authorization",
//     "Access-Control-Allow-Methods",
//     "Access-Control-Request-Headers",
//   ],
//   credentials: true,
//   enablePreflight: true,
// };

// Add the path to receive request and set json as bodyParser to process the body
main.use(cors());
main.use("/api/v1", app);
main.use(express.json());
main.use(express.urlencoded({ extended: false }));

// Initialize the database and the collection
const db = admin.firestore();
const donationsColletion = "donations";
const env = functions.config();

main.post("/", (req, res, next) => {
  void next;

  const { body } = req;

  const gateway = new BraintreeGateway({
    environment: Environment.Sandbox,
    merchantId: env.braintree.merchantid,
    publicKey: env.braintree.publickey,
    privateKey: env.braintree.privatekey,
  });

  try {
    const transaction = gateway.transaction
      .sale({
        amount: body.donationAmount.toFixed(2).toString(),
        paymentMethodNonce: body.nonce,
        options: {
          submitForSettlement: true,
        },
      })
      .then((value) => {
        if (value.success) {
          res.send(value);
          const donationDetails = buildCustomerRecord(value.transaction);
          db.collection(donationsColletion).add(donationDetails);
        } else {
          res.status(500).send({ errors: value.errors.deepErrors() });
        }
      });

    void transaction;
  } catch (e) {
    res.status(500).send({ errors: [e] });
  }
});

// Define google cloud function name
export const paymentsApi = functions.https.onRequest(main);
