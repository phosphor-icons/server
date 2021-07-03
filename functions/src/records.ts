import { Transaction } from "braintree";

interface DonationDetails {
  name: string;
  email?: string;
  amount: number;
  currency: string;
  createdAt: Date | string;
}

export const buildCustomerRecord = (
  transaction: Transaction
): DonationDetails => {
  const firstName =
    transaction.customer.firstName ?? transaction.paypalAccount?.payerFirstName;
  const lastName =
    transaction.customer.lastName ?? transaction.paypalAccount?.payerLastName;
  const fullName = transaction.creditCard?.cardholderName;

  const name =
    fullName || (!!firstName && !!lastName ? `${firstName} ${lastName}` : "");

  const email =
    transaction.customer.email ?? transaction.paypalAccount?.payerEmail ?? "";

  return {
    name,
    email,
    amount: parseFloat(transaction.amount),
    currency: transaction.currencyIsoCode,
    createdAt: transaction.createdAt,
  };
};
