"""
Tests for member events fallback and error handling - F-001 Welcome Message Feature
"""
import pytest
from unittest.mock import AsyncMock, patch
import asyncio

from src.handlers.member_events import MemberEventHandler
from src.services.config_service import ConfigService


class TestMemberEventFallback:
    """Test cases for error handling and fallback scenarios"""
    
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
            }
        }
    
    @pytest.mark.asyncio
    async def test_no_welcome_channel_configured_warning(self, handler, guild_member_add_event):
        """Test appropriate warning when no welcome channel is configured"""
        # Arrange
        handler.config_service.get_welcome_channel.return_value = None
        
        with patch('src.utils.logger.warning') as mock_warning:
            # Act
            await handler.handle_guild_member_add(guild_member_add_event)
            
            # Assert
            mock_warning.assert_called_once_with(
                "No welcome channel configured for guild 123456789"
            )
    
    @pytest.mark.asyncio
    async def test_no_message_sent_without_channel_config(self, handler, guild_member_add_event):
        """Test that no message is sent when channel is not configured"""
        # Arrange
        handler.config_service.get_welcome_channel.return_value = None
        handler._send_welcome_message = AsyncMock()
        
        # Act
        await handler.handle_guild_member_add(guild_member_add_event)
        
        # Assert
        handler._send_welcome_message.assert_not_called()
    
    @pytest.mark.asyncio
    async def test_rate_limit_handling(self, handler, guild_member_add_event):
        """Test handling of Discord API rate limits"""
        # Arrange
        handler.config_service.get_welcome_channel.return_value = '555666777'
        
        # Mock rate limit exception
        class RateLimitError(Exception):
            def __init__(self):
                self.retry_after = 5.0
        
        handler._send_welcome_message = AsyncMock(side_effect=RateLimitError())
        
        with patch('src.utils.logger.warning') as mock_warning:
            with patch('asyncio.sleep') as mock_sleep:
                # Act
                await handler.handle_guild_member_add(guild_member_add_event)
                
                # Assert
                mock_warning.assert_called()
                mock_sleep.assert_called_with(5.0)
    
    @pytest.mark.asyncio 
    async def test_database_connection_error(self, handler, guild_member_add_event):
        """Test handling when config service database is unavailable"""
        # Arrange
        handler.config_service.get_welcome_channel.side_effect = Exception("Connection timeout")
        
        with patch('src.utils.logger.error') as mock_error:
            # Act
            await handler.handle_guild_member_add(guild_member_add_event)
            
            # Assert
            mock_error.assert_called_once()
            error_message = mock_error.call_args[0][0]
            assert "Failed to process member join" in error_message
            assert "123456789" in error_message
    
    @pytest.mark.asyncio
    async def test_malformed_event_data_handling(self, handler):
        """Test handling of malformed event data"""
        # Arrange - missing required fields
        malformed_event = {
            'guild_id': '123456789'
            # Missing 'user' field
        }
        
        with patch('src.utils.logger.error') as mock_error:
            # Act
            await handler.handle_guild_member_add(malformed_event)
            
            # Assert
            mock_error.assert_called_once()
            assert "Invalid event data" in mock_error.call_args[0][0]
    
    @pytest.mark.asyncio
    async def test_channel_not_found_handling(self, handler, guild_member_add_event):
        """Test handling when configured welcome channel doesn't exist"""
        # Arrange
        handler.config_service.get_welcome_channel.return_value = '999999999'  # Non-existent channel
        
        class ChannelNotFoundError(Exception):
            pass
        
        handler._get_channel = AsyncMock(side_effect=ChannelNotFoundError())
        
        with patch('src.utils.logger.error') as mock_error:
            # Act
            await handler.handle_guild_member_add(guild_member_add_event)
            
            # Assert
            mock_error.assert_called_once()
            assert "Welcome channel not found" in mock_error.call_args[0][0]
    
    @pytest.mark.asyncio
    async def test_no_permissions_to_send_message(self, handler, guild_member_add_event):
        """Test handling when bot lacks permissions to send messages"""
        # Arrange
        handler.config_service.get_welcome_channel.return_value = '555666777'
        
        class PermissionError(Exception):
            pass
        
        mock_channel = AsyncMock()
        mock_channel.send = AsyncMock(side_effect=PermissionError())
        handler._get_channel = AsyncMock(return_value=mock_channel)
        
        with patch('src.utils.logger.error') as mock_error:
            # Act
            await handler.handle_guild_member_add(guild_member_add_event)
            
            # Assert
            mock_error.assert_called_once()
            assert "Permission denied" in mock_error.call_args[0][0]