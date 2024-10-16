use crate::errors::impl_from_err_to_app_internal_err;
use aws_sdk_dynamodb::config::http::HttpResponse;
use aws_sdk_dynamodb::error::SdkError;
use aws_sdk_dynamodb::operation::batch_get_item::BatchGetItemError;
use aws_sdk_dynamodb::operation::batch_write_item::BatchWriteItemError;
use aws_sdk_dynamodb::operation::delete_item::DeleteItemError;
use aws_sdk_dynamodb::operation::get_item::GetItemError;
use aws_sdk_dynamodb::operation::put_item::PutItemError;
use aws_sdk_dynamodb::operation::query::QueryError;
use aws_sdk_dynamodb::operation::transact_write_items::TransactWriteItemsError;

impl_from_err_to_app_internal_err!(SdkError<GetItemError, HttpResponse>);
impl_from_err_to_app_internal_err!(SdkError<PutItemError, HttpResponse>);
impl_from_err_to_app_internal_err!(SdkError<DeleteItemError, HttpResponse>);
impl_from_err_to_app_internal_err!(SdkError<QueryError, HttpResponse>);
impl_from_err_to_app_internal_err!(SdkError<BatchGetItemError, HttpResponse>);
impl_from_err_to_app_internal_err!(SdkError<BatchWriteItemError, HttpResponse>);
impl_from_err_to_app_internal_err!(SdkError<TransactWriteItemsError, HttpResponse>);
