use prost::Message;

#[derive(Message, validify::Validify)]
struct Data {
    #[validate(range(min = 5.0, max = 10.0))]
    #[prost(int32, tag = "1")]
    v0: i32,

    #[prost(string, tag = "2")]
    #[modify(lowercase)]
    #[validate(length(min = 1, max = 10))]
    v1: String,
}
