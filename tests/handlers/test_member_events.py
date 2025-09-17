"""
Tests for member events handler - F-001 Welcome Message Feature
"""
import pytest
import asyncio
from unittest.mock import AsyncMock, MagicMock, patch
from datetime import datetime, timedelta

# Assuming these will be the actual imports once implemented
from src.handlers.member_events import MemberEventHandler
from src.services.config_service import ConfigService


class TestMemberEventHandler:
    """Test cases for GUILD_MEMBER_ADD event handling"""
    
    @pytest.fixture
    async def handler(self):
        """Create handler instance with mocked dependencies"""
        config_service = AsyncMock(spec=ConfigService)
        handler = MemberEventHandler(config_service)
        return handler
    
    @pytest.fixture
    def guild_member_add_event(self):
        """Mock GUILD_MEMBER_ADD event data"""
        return {
            'guild_id': '123456789',
            'user': {
                'id': '987654321',
                'username': 'new_user',
                'discriminator': '1234'
            },
            'joined_at': datetime.utcnow().isoformat()
        }
    
    @pytest.mark.asyncio
    async def test_handle_member_join_success(self, handler, guild_member_add_event):
        """Test successful welcome message sending"""
        # Arrange
        welcome_channel_id = '555666777'
        handler.config_service.get_welcome_channel.return_value = welcome_channel_id
        handler._send_welcome_message = AsyncMock()
        
        # Act
        await handler.handle_guild_member_add(guild_member_add_event)
        
        # Assert
        handler.config_service.get_welcome_channel.assert_called_once_with('123456789')
        handler._send_welcome_message.assert_called_once_with(
            welcome_channel_id, 
            guild_member_add_event['user']
        )
    
    @pytest.mark.asyncio
    async def test_handle_member_join_no_welcome_channel(self, handler, guild_member_add_event):
        """Test handling when welcome channel is not configured"""
        # Arrange
        handler.config_service.get_welcome_channel.return_value = None
        handler._send_welcome_message = AsyncMock()
        
        with patch('src.utils.logger.warning') as mock_warning:
            # Act
            await handler.handle_guild_member_add(guild_member_add_event)
            
            # Assert
            handler.config_service.get_welcome_channel.assert_called_once_with('123456789')
            handler._send_welcome_message.assert_not_called()
            mock_warning.assert_called_once_with(
                "No welcome channel configured for guild 123456789"
            )
    
    @pytest.mark.asyncio
    async def test_send_welcome_message_with_mention(self, handler):
        """Test welcome message includes correct user mention"""
        # Arrange
        channel_id = '555666777'
        user_data = {
            'id': '987654321',
            'username': 'new_user'
        }
        
        mock_channel = AsyncMock()
        mock_channel.send = AsyncMock()
        handler._get_channel = AsyncMock(return_value=mock_channel)
        
        # Act
        await handler._send_welcome_message(channel_id, user_data)
        
        # Assert
        handler._get_channel.assert_called_once_with(channel_id)
        mock_channel.send.assert_called_once()
        
        # Verify message contains user mention
        call_args = mock_channel.send.call_args
        message_content = call_args[1]['content'] if 'content' in call_args[1] else call_args[0][0]
        assert '<@987654321>' in message_content
    
    @pytest.mark.asyncio
    async def test_performance_requirement_3_seconds(self, handler, guild_member_add_event):
        """Test that welcome message is sent within 3 seconds (NFR-P-002)"""
        # Arrange
        handler.config_service.get_welcome_channel.return_value = '555666777'
        handler._send_welcome_message = AsyncMock()
        
        # Act
        start_time = datetime.utcnow()
        await handler.handle_guild_member_add(guild_member_add_event)
        end_time = datetime.utcnow()
        
        # Assert
        elapsed = (end_time - start_time).total_seconds()
        assert elapsed < 3.0, f"Welcome message took {elapsed:.2f}s, should be < 3s"
    
    @pytest.mark.asyncio
    async def test_config_service_error_handling(self, handler, guild_member_add_event):
        """Test handling of config service errors"""
        # Arrange
        handler.config_service.get_welcome_channel.side_effect = Exception("Database error")
        
        with patch('src.utils.logger.error') as mock_error:
            # Act
            await handler.handle_guild_member_add(guild_member_add_event)
            
            # Assert
            mock_error.assert_called_once()
            assert "Failed to process member join" in mock_error.call_args[0][0]
    
    @pytest.mark.asyncio
    async def test_discord_api_error_handling(self, handler, guild_member_add_event):
        """Test handling of Discord API errors when sending message"""
        # Arrange
        handler.config_service.get_welcome_channel.return_value = '555666777'
        handler._send_welcome_message = AsyncMock(side_effect=Exception("Discord API error"))
        
        with patch('src.utils.logger.error') as mock_error:
            # Act
            await handler.handle_guild_member_add(guild_member_add_event)
            
            # Assert
            mock_error.assert_called_once()
    
    @pytest.mark.asyncio
    async def test_duplicate_event_handling(self, handler, guild_member_add_event):
        """Test idempotency - same event should not trigger duplicate messages"""
        # This will be implemented with dedup/idempotency component
        # Arrange
        handler.config_service.get_welcome_channel.return_value = '555666777'
        handler._send_welcome_message = AsyncMock()
        handler._is_duplicate_event = MagicMock(return_value=True)
        
        # Act
        await handler.handle_guild_member_add(guild_member_add_event)
        
        # Assert
        handler._send_welcome_message.assert_not_called()